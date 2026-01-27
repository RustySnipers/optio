import { useState, useEffect } from "react";
import {
    Users,
    Plus,
    Search,
    Mail,
    Network,
    Calendar,
    X,
    Loader2,
    Trash2,
} from "lucide-react";
import {
    listClients,
    createClient,
    deleteClient,
} from "@/lib/commands";
import type { Client, CreateClientRequest } from "@/types";

export function ClientManagement() {
    const [clients, setClients] = useState<Client[]>([]);
    const [isLoading, setIsLoading] = useState(true);
    const [searchQuery, setSearchQuery] = useState("");
    const [showNewClientModal, setShowNewClientModal] = useState(false);
    const [isSubmitting, setIsSubmitting] = useState(false);

    // New client form state
    const [newClient, setNewClient] = useState<CreateClientRequest>({
        name: "",
        targetSubnet: "",
        contactEmail: "",
        notes: ""
    });

    const loadClients = async () => {
        setIsLoading(true);
        try {
            const data = await listClients();
            setClients(data);
        } catch (error) {
            console.error("Failed to load clients:", error);
        } finally {
            setIsLoading(false);
        }
    };

    useEffect(() => {
        loadClients();
    }, []);

    const handleCreateClient = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!newClient.name) return;

        setIsSubmitting(true);
        try {
            await createClient(newClient);
            await loadClients();
            setShowNewClientModal(false);
            setNewClient({ name: "", targetSubnet: "", contactEmail: "", notes: "" });
        } catch (error) {
            console.error("Failed to create client:", error);
        } finally {
            setIsSubmitting(false);
        }
    };

    const handleDeleteClient = async (id: string, e: React.MouseEvent) => {
        e.stopPropagation();
        if (!confirm("Are you sure you want to delete this client?")) return;

        try {
            await deleteClient(id);
            await loadClients();
        } catch (error) {
            console.error("Failed to delete client:", error);
        }
    };

    const filteredClients = clients.filter(client =>
        client.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        (client.contactEmail && client.contactEmail.toLowerCase().includes(searchQuery.toLowerCase()))
    );

    return (
        <div className="h-full flex flex-col p-6">
            {/* Header */}
            <div className="flex items-center justify-between mb-8">
                <div className="flex items-center gap-3">
                    <div className="p-2 bg-blue-600/20 rounded-lg">
                        <Users className="w-6 h-6 text-blue-400" />
                    </div>
                    <div>
                        <h1 className="text-2xl font-bold text-white">Client Management</h1>
                        <p className="text-slate-400">Manage client profiles and configurations</p>
                    </div>
                </div>
                <button
                    onClick={() => setShowNewClientModal(true)}
                    className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors font-medium"
                >
                    <Plus className="w-5 h-5" />
                    New Client
                </button>
            </div>

            {/* Filters */}
            <div className="mb-6">
                <div className="relative max-w-md">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-500" />
                    <input
                        type="text"
                        placeholder="Search clients..."
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        className="w-full pl-10 pr-4 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                </div>
            </div>

            {/* Client List */}
            <div className="flex-1 overflow-y-auto">
                {isLoading ? (
                    <div className="flex items-center justify-center h-64">
                        <Loader2 className="w-8 h-8 text-blue-500 animate-spin" />
                    </div>
                ) : filteredClients.length > 0 ? (
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                        {filteredClients.map((client) => (
                            <div
                                key={client.id}
                                className="bg-slate-800 border border-slate-700 rounded-xl p-6 hover:border-slate-600 transition-colors group relative"
                            >
                                <div className="absolute top-4 right-4 opacity-0 group-hover:opacity-100 transition-opacity">
                                    <button
                                        onClick={(e) => handleDeleteClient(client.id, e)}
                                        className="p-2 hover:bg-slate-700 rounded-lg text-slate-400 hover:text-red-400 transition-colors"
                                    >
                                        <Trash2 className="w-4 h-4" />
                                    </button>
                                </div>

                                <div className="flex items-start justify-between mb-4">
                                    <div className="w-12 h-12 bg-slate-700 rounded-full flex items-center justify-center text-xl font-bold text-white">
                                        {client.name.charAt(0).toUpperCase()}
                                    </div>
                                </div>

                                <h3 className="text-lg font-semibold text-white mb-1">{client.name}</h3>
                                <p className="text-sm text-slate-400 mb-4 line-clamp-2">
                                    {client.notes || "No notes provided"}
                                </p>

                                <div className="space-y-2 text-sm">
                                    {client.contactEmail && (
                                        <div className="flex items-center gap-2 text-slate-300">
                                            <Mail className="w-4 h-4 text-slate-500" />
                                            {client.contactEmail}
                                        </div>
                                    )}
                                    {client.targetSubnet && (
                                        <div className="flex items-center gap-2 text-slate-300">
                                            <Network className="w-4 h-4 text-slate-500" />
                                            {client.targetSubnet}
                                        </div>
                                    )}
                                    <div className="flex items-center gap-2 text-slate-300">
                                        <Calendar className="w-4 h-4 text-slate-500" />
                                        Added {new Date(client.createdAt).toLocaleDateString()}
                                    </div>
                                </div>
                            </div>
                        ))}
                    </div>
                ) : (
                    <div className="text-center py-20 bg-slate-800/50 rounded-xl border border-dashed border-slate-700">
                        <Users className="w-12 h-12 text-slate-600 mx-auto mb-4" />
                        <h3 className="text-lg font-medium text-white mb-2">No clients found</h3>
                        <p className="text-slate-400 max-w-sm mx-auto mb-6">
                            Get started by adding your first client profile to manage configurations and reports.
                        </p>
                        <button
                            onClick={() => setShowNewClientModal(true)}
                            className="inline-flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors"
                        >
                            <Plus className="w-4 h-4" />
                            Add Client
                        </button>
                    </div>
                )}
            </div>

            {/* New Client Modal */}
            {showNewClientModal && (
                <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm">
                    <div className="bg-slate-900 border border-slate-700 rounded-xl w-full max-w-lg shadow-2xl">
                        <div className="p-6 border-b border-slate-800 flex items-center justify-between">
                            <h2 className="text-xl font-bold text-white">Add New Client</h2>
                            <button
                                onClick={() => setShowNewClientModal(false)}
                                className="text-slate-400 hover:text-white transition-colors"
                            >
                                <X className="w-5 h-5" />
                            </button>
                        </div>

                        <form onSubmit={handleCreateClient} className="p-6 space-y-4">
                            <div>
                                <label className="block text-sm font-medium text-slate-300 mb-1">
                                    Client Name *
                                </label>
                                <input
                                    type="text"
                                    required
                                    value={newClient.name}
                                    onChange={(e) => setNewClient({ ...newClient, name: e.target.value })}
                                    className="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
                                    placeholder="e.g. Acme Corp"
                                />
                            </div>

                            <div>
                                <label className="block text-sm font-medium text-slate-300 mb-1">
                                    Target Subnet
                                </label>
                                <input
                                    type="text"
                                    value={newClient.targetSubnet || ""}
                                    onChange={(e) => setNewClient({ ...newClient, targetSubnet: e.target.value })}
                                    className="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
                                    placeholder="e.g. 192.168.1.0/24"
                                />
                            </div>

                            <div>
                                <label className="block text-sm font-medium text-slate-300 mb-1">
                                    Contact Email
                                </label>
                                <input
                                    type="email"
                                    value={newClient.contactEmail || ""}
                                    onChange={(e) => setNewClient({ ...newClient, contactEmail: e.target.value })}
                                    className="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
                                    placeholder="admin@acme.com"
                                />
                            </div>

                            <div>
                                <label className="block text-sm font-medium text-slate-300 mb-1">
                                    Notes
                                </label>
                                <textarea
                                    value={newClient.notes || ""}
                                    onChange={(e) => setNewClient({ ...newClient, notes: e.target.value })}
                                    className="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500 h-24 resize-none"
                                    placeholder="Additional details..."
                                />
                            </div>

                            <div className="flex justify-end gap-3 pt-4">
                                <button
                                    type="button"
                                    onClick={() => setShowNewClientModal(false)}
                                    className="px-4 py-2 text-slate-300 hover:text-white hover:bg-slate-800 rounded-lg transition-colors"
                                >
                                    Cancel
                                </button>
                                <button
                                    type="submit"
                                    disabled={isSubmitting}
                                    className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors disabled:opacity-50 flex items-center gap-2"
                                >
                                    {isSubmitting ? (
                                        <>
                                            <Loader2 className="w-4 h-4 animate-spin" />
                                            Creating...
                                        </>
                                    ) : (
                                        "Create Client"
                                    )}
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            )}
        </div>
    );
}
